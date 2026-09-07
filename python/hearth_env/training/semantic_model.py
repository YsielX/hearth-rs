"""Shared effect-language encoder and public-state relational encoder."""

from __future__ import annotations

from dataclasses import replace
from typing import Any

import torch
from torch import nn

from .semantics import (
    CLASSES,
    EVENT_KINDS,
    HISTORY_GROUPS,
    KEYWORDS,
    KINDS,
    ROLES,
    STATIC_DIM,
    ZONES,
    effect_tokens,
    static_features,
    vocabulary,
)


class EffectTextEncoder(nn.Module):
    """Local effect phrases retain word order and bind numbers to their words."""

    def __init__(self, vocab_size: int, width: int, hidden: int) -> None:
        super().__init__()
        self.words = nn.Embedding(vocab_size, width, padding_idx=0)
        self.numbers = nn.Linear(2, width, bias=False)
        self.phrases = nn.ModuleList(
            nn.Conv1d(width, width, kernel, padding=kernel // 2) for kernel in (1, 3, 5)
        )
        self.output = nn.Sequential(
            nn.Linear(width * 6, hidden), nn.GELU(), nn.LayerNorm(hidden)
        )

    def forward(self, tokens: torch.Tensor, numbers: torch.Tensor) -> torch.Tensor:
        mask = tokens.ne(0)
        words = (self.words(tokens) + self.numbers(numbers)) * mask.unsqueeze(-1)
        phrase_outputs = []
        for conv in self.phrases:
            phrase = torch.nn.functional.gelu(conv(words.transpose(1, 2))).transpose(
                1, 2
            )
            mean = (phrase * mask.unsqueeze(-1)).sum(1) / mask.sum(
                1, keepdim=True
            ).clamp_min(1)
            maximum = phrase.masked_fill(
                ~mask.unsqueeze(-1), torch.finfo(phrase.dtype).min
            ).amax(1)
            maximum = torch.where(mask.any(1, keepdim=True), maximum, 0)
            phrase_outputs.extend((mean, maximum))
        return self.output(torch.cat(phrase_outputs, -1))


class SemanticEncoder:
    """Mixin used by HearthQNetwork; legacy parameters stay separately loadable."""

    def _init_semantic(self, catalog: Any, config: Any) -> None:
        from .tensorize import ACTION_KINDS

        self.config = replace(
            config,
            text_vocabulary=config.text_vocabulary or vocabulary(catalog.entries),
        )
        config = self.config
        hidden = config.hidden_dim
        indices = {word: i for i, word in enumerate(config.text_vocabulary)}
        token_table = torch.zeros(
            len(catalog.card_ids), config.max_card_tokens, dtype=torch.long
        )
        number_table = torch.zeros(len(catalog.card_ids), config.max_card_tokens, 2)
        feature_table = torch.zeros(len(catalog.card_ids), STATIC_DIM)
        for i, card in enumerate(catalog.card_ids[2:], 2):
            definition = catalog.entries[card]["definition"]
            tokens = effect_tokens(str(definition.get("text") or ""))
            if len(tokens) > config.max_card_tokens:
                raise ValueError(
                    f"card text exceeds max_card_tokens={config.max_card_tokens}: {card} ({len(tokens)})"
                )
            for j, (word, numbers) in enumerate(tokens):
                token_table[i, j] = indices.get(word, 1)
                number_table[i, j] = torch.tensor(numbers)
            feature_table[i] = torch.tensor(static_features(definition))
        used_tokens = max(int(token_table.ne(0).sum(1).max()), 1)
        token_table = token_table[:, :used_tokens]
        number_table = number_table[:, :used_tokens]
        self.register_buffer("card_feature_table", feature_table, persistent=False)
        self.register_buffer("card_token_table", token_table, persistent=False)
        self.register_buffer("card_number_table", number_table, persistent=False)
        self.card_text = EffectTextEncoder(len(indices), config.text_dim, hidden)
        self.card_semantic = nn.Sequential(
            nn.Linear(STATIC_DIM, hidden), nn.GELU(), nn.LayerNorm(hidden)
        )
        # Kept as an inert migration surface, never used by the semantic policy.
        self.card_id_embedding = nn.Embedding(
            len(catalog.card_ids), hidden, padding_idx=0
        )
        nn.init.zeros_(self.card_id_embedding.weight)
        self.card_id_embedding.weight.requires_grad_(False)
        self._card_cache = None
        self._card_cache_signature = None

        self.entity_state = nn.Sequential(
            nn.Linear(config.entity_state_dim + len(KEYWORDS) + len(KINDS), hidden),
            nn.GELU(),
        )
        layer = nn.TransformerEncoderLayer(
            hidden,
            config.attention_heads,
            hidden * 3,
            config.dropout,
            batch_first=True,
            norm_first=True,
        )
        self.entity_encoder = nn.TransformerEncoder(
            layer, config.transformer_layers, enable_nested_tensor=False
        )
        self.global_encoder = nn.Sequential(
            nn.Linear(config.global_dim + len(CLASSES) * 2, hidden),
            nn.GELU(),
            nn.LayerNorm(hidden),
        )
        self.history_kind = nn.Embedding(len(EVENT_KINDS) + 1, hidden)
        self.history_role = nn.Embedding(len(ROLES) + 1, hidden, padding_idx=0)
        self.history_zone = nn.Embedding(len(ZONES) + 1, hidden, padding_idx=0)
        self.history_participant = nn.Sequential(
            nn.Linear(hidden * 3 + 1, hidden), nn.GELU()
        )
        self.history_numeric = nn.Linear(
            config.history_numeric_dim + len(KEYWORDS), hidden
        )
        self.history_encoder = nn.TransformerEncoder(
            layer, 1, enable_nested_tensor=False
        )
        self.memory_group = nn.Embedding(len(HISTORY_GROUPS) * 2, hidden)
        self.memory_item = nn.Sequential(nn.Linear(hidden * 2 + 2, hidden), nn.GELU())
        self.fact_text = EffectTextEncoder(257, 16, hidden)
        self.fact_item = nn.Sequential(nn.Linear(hidden * 2 + 3, hidden), nn.GELU())
        self.context = nn.Sequential(
            nn.Linear(hidden * 5, hidden * 2), nn.GELU(), nn.LayerNorm(hidden * 2)
        )
        self.action_kind = nn.Embedding(len(ACTION_KINDS), hidden)
        self.action_numeric = nn.Linear(config.action_numeric_dim, hidden)
        self.action_scorer = nn.Sequential(
            nn.Linear(hidden * 7, hidden * 2),
            nn.GELU(),
            nn.Dropout(config.dropout),
            nn.Linear(hidden * 2, hidden),
            nn.GELU(),
            nn.Linear(hidden, 1),
        )
        self.value_head = nn.Sequential(
            nn.Linear(hidden * 2, hidden), nn.GELU(), nn.Linear(hidden, 1)
        )
        nn.init.zeros_(self.value_head[-1].weight)
        nn.init.zeros_(self.value_head[-1].bias)

    def _semantic_card_table(
        self, used_cards: torch.Tensor | None = None
    ) -> torch.Tensor:
        if used_cards is not None:
            indices = used_cards.unique()
            values = self.card_semantic(
                self.card_feature_table[indices]
            ) + self.card_text(
                self.card_token_table[indices], self.card_number_table[indices]
            )
            values = values * indices.gt(0).unsqueeze(-1)
            return values.new_zeros(
                self.card_feature_table.shape[0], self.config.hidden_dim
            ).index_copy(0, indices, values)
        parameters = (*self.card_text.parameters(), *self.card_semantic.parameters())
        device_type = self.card_feature_table.device.type
        autocast = torch.is_autocast_enabled(device_type)
        signature = (
            self.card_feature_table.device,
            self.card_feature_table.dtype,
            torch.get_autocast_dtype(device_type) if autocast else None,
            tuple(p._version for p in parameters),
        )
        caching = not self.training and not torch.is_grad_enabled()
        if (
            caching
            and signature == self._card_cache_signature
            and self._card_cache is not None
        ):
            return self._card_cache
        values = self.card_semantic(self.card_feature_table) + self.card_text(
            self.card_token_table, self.card_number_table
        )
        values = values * torch.arange(values.shape[0], device=values.device).gt(
            0
        ).unsqueeze(-1)
        if caching:
            self._card_cache, self._card_cache_signature = values, signature
        return values

    @staticmethod
    def _references(entity: torch.Tensor, refs: torch.Tensor) -> torch.Tensor:
        batch = torch.arange(entity.shape[0], device=entity.device).reshape(
            -1, *([1] * (refs.ndim - 1))
        )
        return entity[batch, refs.clamp_min(0)] * refs.ge(0).unsqueeze(-1)

    def _encode_semantic_state(
        self, batch: dict[str, torch.Tensor]
    ) -> tuple[torch.Tensor, torch.Tensor]:
        # Compute static semantics once per forward, not once per occurrence.
        used_cards = None
        if torch.is_grad_enabled() or self.training:
            # Card encoders act independently on each row. Unreferenced rows
            # have zero gradients, so avoid evaluating them during learning.
            used_cards = torch.cat(
                [
                    batch[key].flatten()
                    for key in (
                        "entity_cards",
                        "entity_public_cards",
                        "deck_cards",
                        "history_cards",
                        "memory_cards",
                        "action_semantic_cards",
                    )
                ]
            )
        table = self._semantic_card_table(used_cards)
        card = lambda key: table[batch[key]]
        public = (
            card("entity_public_cards") * batch["entity_public_card_mask"].unsqueeze(-1)
        ).sum(2)
        entity = (
            card("entity_cards")
            + public
            + self.entity_state(
                torch.cat(
                    (
                        batch["entity_state"],
                        batch["entity_keywords"],
                        batch["entity_kinds"],
                    ),
                    -1,
                )
            )
        )
        global_context = self.global_encoder(
            torch.cat((batch["global_state"], batch["player_classes"].flatten(1)), -1)
        )
        deck_context = self._masked_mean(card("deck_cards"), batch["deck_mask"])
        state_tokens = torch.cat(
            (entity, global_context[:, None], deck_context[:, None]), 1
        )
        state_mask = torch.cat(
            (
                batch["entity_mask"],
                torch.ones(entity.shape[0], 2, dtype=torch.bool, device=entity.device),
            ),
            1,
        )
        state_tokens = self.entity_encoder(
            state_tokens, src_key_padding_mask=~state_mask
        )
        entity = state_tokens[:, : entity.shape[1]]
        entity_context = self._masked_mean(state_tokens, state_mask)

        participant = self.history_participant(
            torch.cat(
                (
                    card("history_cards"),
                    self.history_role(batch["history_roles"]),
                    self._references(entity, batch["history_refs"]),
                    batch["history_entity_values"],
                ),
                -1,
            )
        )
        participant_mask = batch["history_card_mask"].unsqueeze(-1)
        event_cards = (participant * participant_mask).sum(2) / participant_mask.sum(
            2
        ).clamp_min(1)
        history = (
            self.history_kind(batch["history_kinds"])
            + event_cards
            + self.history_numeric(
                torch.cat((batch["history_numeric"], batch["history_keywords"]), -1)
            )
        )
        # Preserve from/to direction rather than summing interchangeable zones.
        zones = self.history_zone(batch["history_zones"])
        history = history + zones[:, :, 0] - zones[:, :, 1]
        history = self.history_encoder(
            history, src_key_padding_mask=~batch["history_mask"]
        )
        history_context = self._masked_mean(history, batch["history_mask"])

        counts = batch["memory_counts"]
        memory = self.memory_item(
            torch.cat(
                (
                    card("memory_cards"),
                    self.memory_group(batch["memory_groups"]),
                    (counts / 10).unsqueeze(-1),
                    counts.log1p().unsqueeze(-1),
                ),
                -1,
            )
        )
        memory_context = (memory * batch["memory_mask"].unsqueeze(-1)).sum(1) / 20

        fact_bytes = batch["fact_bytes"]
        flat_bytes = fact_bytes.flatten(0, 1)
        facts = self.fact_text(
            flat_bytes, torch.zeros(*flat_bytes.shape, 2, device=entity.device)
        ).reshape(entity.shape[0], fact_bytes.shape[1], -1)
        facts = self.fact_item(
            torch.cat(
                (
                    facts,
                    self._references(entity, batch["fact_entities"]),
                    batch["fact_values"],
                ),
                -1,
            )
        )
        memory_context = (
            memory_context + (facts * batch["fact_mask"].unsqueeze(-1)).sum(1) / 10
        )

        # A discovered option set is part of the state, including for the critic.
        options = card("action_semantic_cards")
        option_mask = batch["action_semantic_card_mask"] & batch[
            "action_mask"
        ].unsqueeze(-1)
        options = (options * option_mask.unsqueeze(-1)).sum((1, 2)) / option_mask.sum(
            (1, 2)
        ).clamp_min(1).unsqueeze(-1)
        context = self.context(
            torch.cat(
                (
                    entity_context,
                    history_context,
                    memory_context,
                    deck_context,
                    options,
                ),
                -1,
            )
        )
        # policy_value also encodes action semantics; reuse this differentiable table.
        self._forward_card_table = table
        return entity, context
