from __future__ import annotations

import torch
from torch import nn

from .catalog import CardCatalog
from .config import ModelConfig
from .tensorize import ACTION_KINDS


class HearthQNetwork(nn.Module):
    """One shared state encoder followed by a score for every legal action."""

    def __init__(self, catalog: CardCatalog, config: ModelConfig) -> None:
        super().__init__()
        self.config = config
        hidden = config.hidden_dim
        features = torch.tensor(catalog.feature_matrix(), dtype=torch.float32)
        self.register_buffer("card_feature_table", features, persistent=False)
        self.card_id_embedding = nn.Embedding(
            len(catalog.card_ids), hidden, padding_idx=0
        )
        self.card_semantic = nn.Sequential(
            nn.Linear(catalog.feature_dim, hidden), nn.GELU(), nn.LayerNorm(hidden)
        )
        self.entity_state = nn.Sequential(
            nn.Linear(config.entity_state_dim, hidden), nn.GELU()
        )
        layer = nn.TransformerEncoderLayer(
            hidden,
            config.attention_heads,
            hidden * 4,
            config.dropout,
            batch_first=True,
            norm_first=True,
        )
        self.entity_encoder = nn.TransformerEncoder(
            layer, config.transformer_layers, enable_nested_tensor=False
        )
        self.history_kind = nn.Embedding(256, hidden)
        self.history_numeric = nn.Linear(config.history_numeric_dim, hidden)
        history_layer = nn.TransformerEncoderLayer(
            hidden,
            config.attention_heads,
            hidden * 4,
            config.dropout,
            batch_first=True,
            norm_first=True,
        )
        self.history_encoder = nn.TransformerEncoder(
            history_layer, 1, enable_nested_tensor=False
        )
        self.global_encoder = nn.Sequential(
            nn.Linear(config.global_dim, hidden), nn.GELU(), nn.LayerNorm(hidden)
        )
        self.context = nn.Sequential(
            nn.Linear(hidden * 4, hidden * 2),
            nn.GELU(),
            nn.LayerNorm(hidden * 2),
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
            nn.Linear(hidden * 2, hidden),
            nn.GELU(),
            nn.Linear(hidden, 1),
        )
        # A BC checkpoint has no value target. Starting at zero is a stable,
        # honest prior when such a checkpoint is promoted into PPO training.
        nn.init.zeros_(self.value_head[-1].weight)
        nn.init.zeros_(self.value_head[-1].bias)

    def encode_card(self, indices: torch.Tensor) -> torch.Tensor:
        safe = indices.clamp(0, self.card_feature_table.shape[0] - 1)
        semantic = self.card_semantic(self.card_feature_table[safe])
        return semantic + self.card_id_embedding(safe)

    @staticmethod
    def _masked_mean(values: torch.Tensor, mask: torch.Tensor) -> torch.Tensor:
        weights = mask.to(values.dtype).unsqueeze(-1)
        return (values * weights).sum(dim=1) / weights.sum(dim=1).clamp_min(1.0)

    def _encode_state(
        self, batch: dict[str, torch.Tensor]
    ) -> tuple[torch.Tensor, torch.Tensor]:
        public_card_values = self.encode_card(batch["entity_public_cards"])
        public_card_weights = batch["entity_public_card_mask"].to(
            public_card_values.dtype
        ).unsqueeze(-1)
        # Keep multiplicity: two identical Starship pieces are semantically
        # different from one even though their mean embedding is identical.
        public_card_context = (public_card_values * public_card_weights).sum(2)
        entity = (
            self.encode_card(batch["entity_cards"])
            + self.entity_state(batch["entity_state"])
            + public_card_context
        )
        entity = self.entity_encoder(entity, src_key_padding_mask=~batch["entity_mask"])
        entity_context = self._masked_mean(entity, batch["entity_mask"])

        history_card_values = self.encode_card(batch["history_cards"])
        history_card_weights = (
            batch["history_card_mask"].to(history_card_values.dtype).unsqueeze(-1)
        )
        history_card_context = (history_card_values * history_card_weights).sum(
            2
        ) / history_card_weights.sum(2).clamp_min(1)
        history = (
            self.history_kind(batch["history_kinds"])
            + history_card_context
            + self.history_numeric(batch["history_numeric"])
        )
        history = self.history_encoder(
            history, src_key_padding_mask=~batch["history_mask"]
        )
        history_context = self._masked_mean(history, batch["history_mask"])
        deck_context = self._masked_mean(
            self.encode_card(batch["deck_cards"]), batch["deck_mask"]
        )
        global_context = self.global_encoder(batch["global_state"])
        context = self.context(
            torch.cat(
                [global_context, entity_context, history_context, deck_context], dim=-1
            )
        )
        return entity, context

    def policy_value(
        self, batch: dict[str, torch.Tensor]
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """Return masked policy logits and the actor-relative state value."""

        entity, context = self._encode_state(batch)

        _, action_count, _ = batch["action_sources"].shape
        safe_sources = batch["action_sources"].clamp_min(0)
        expanded_entities = entity.unsqueeze(1).expand(-1, action_count, -1, -1)
        source_values = torch.gather(
            expanded_entities,
            2,
            safe_sources.unsqueeze(-1).expand(-1, -1, -1, entity.shape[-1]),
        )
        source_weights = batch["action_source_mask"].to(entity.dtype).unsqueeze(-1)
        source_context = (source_values * source_weights).sum(2) / source_weights.sum(
            2
        ).clamp_min(1)

        safe_targets = batch["action_targets"].clamp_min(0)
        target_context = torch.gather(
            entity,
            1,
            safe_targets.unsqueeze(-1).expand(-1, -1, entity.shape[-1]),
        )
        has_target = batch["action_targets"].ge(0).unsqueeze(-1)
        target_context = target_context * has_target

        action_card_values = self.encode_card(batch["action_semantic_cards"])
        action_card_weights = batch["action_semantic_card_mask"].to(
            action_card_values.dtype
        ).unsqueeze(-1)
        action_card_context = (action_card_values * action_card_weights).sum(
            2
        ) / action_card_weights.sum(2).clamp_min(1)

        action = torch.cat(
            [
                context.unsqueeze(1).expand(-1, action_count, -1),
                self.action_kind(batch["action_kinds"]),
                self.action_numeric(batch["action_numeric"]),
                source_context,
                target_context,
                action_card_context,
            ],
            dim=-1,
        )
        logits = self.action_scorer(action).squeeze(-1)
        logits = logits.masked_fill(
            ~batch["action_mask"], torch.finfo(logits.dtype).min
        )
        value = torch.tanh(self.value_head(context).squeeze(-1))
        return logits, value

    def forward(self, batch: dict[str, torch.Tensor]) -> torch.Tensor:
        logits, _ = self.policy_value(batch)
        return logits

    def extra_repr(self) -> str:
        return f"hidden_dim={self.config.hidden_dim}"


def selected_q(q_values: torch.Tensor, actions: torch.Tensor) -> torch.Tensor:
    return q_values.gather(1, actions.unsqueeze(1)).squeeze(1)


def parameter_count(model: nn.Module) -> int:
    return sum(parameter.numel() for parameter in model.parameters())
