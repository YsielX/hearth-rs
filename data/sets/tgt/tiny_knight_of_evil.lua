return {
    api_version = 1, id = "AT_021", name = "Tiny Knight of Evil",
    text = "Whenever you discard a card, gain +2/+1.", set = "TGT", type = "minion",
    class = "warlock", rarity = "rare", cost = 2, attack = 3, health = 2, tags = { "demon" },
    triggers = {{
        event = "card_discarded", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 2, 1) end,
    }},
}
