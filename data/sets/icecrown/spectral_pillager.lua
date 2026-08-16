local card = {
    api_version = 1, id = "ICC_910", name = "Spectral Pillager",
    text = "[x]<b>Combo:</b> Deal 2 damage to\na minion for each other card\nyou've played this turn.",
    set = "ICECROWN", type = "minion", class = "rogue", rarity = "epic",
    cost = 5, attack = 5, health = 5, tags = { "undead" }, keywords = { "combo" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        if not ctx:combo_active(self) then return {} end
        return ctx:minions()
    end,
}

function card.on_combo(ctx, self, target)
    if target ~= nil then cardlib.effects.damage(ctx, target, 2 * ctx:entity(self).cards_played_before) end
end

return card
