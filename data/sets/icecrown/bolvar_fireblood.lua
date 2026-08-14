local card = {
    api_version = 1, id = "ICC_858", name = "Bolvar, Fireblood",
    text = "<b>Divine Shield</b>\nAfter a friendly minion loses <b>Divine Shield</b>, gain +2 Attack.",
    set = "ICECROWN", type = "minion", class = "paladin", rarity = "legendary",
    cost = 5, attack = 1, health = 7, tags = { "undead" }, keywords = { "divine_shield" },
}

card.triggers = {{
    event = "keyword_disabled", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        local target = ctx:entity(event.target)
        return event.keyword == "divine_shield" and target.type == "minion"
            and target.controller == ctx:controller(self)
    end,
    effect = function(ctx, self) ctx:buff(self, 2, 0) end,
}}

return card
