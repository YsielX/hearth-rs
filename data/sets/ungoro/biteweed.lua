return {
    api_version = 1, id = "UNG_063", name = "Biteweed",
    text = "<b>Combo:</b> Gain +1/+1 for each other card you've played this turn.",
    set = "UNGORO", type = "minion", class = "rogue", rarity = "epic",
    cost = 2, attack = 1, health = 2, keywords = { "combo" },
    on_combo = function(ctx, self)
        local count = ctx:entity(self).cards_played_before
        if count > 0 then cardlib.effects.buff(ctx, self, count, count) end
    end,
}
