local card = {
    api_version = 1, id = "ICC_071", name = "Light's Sorrow",
    text = "After a friendly minion loses <b>Divine Shield</b>, gain +1 Attack.",
    set = "ICECROWN", type = "weapon", class = "paladin", rarity = "epic",
    cost = 4, attack = 1, health = 4,
}

card.triggers = {{
    event = "keyword_disabled", timing = "after", active_zones = { "weapon" },
    condition = function(ctx, self, event)
        local target = ctx:entity(event.target)
        return event.keyword == "divine_shield" and target.type == "minion"
            and target.controller == ctx:controller(self)
    end,
    effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 0) end,
}}

return card
