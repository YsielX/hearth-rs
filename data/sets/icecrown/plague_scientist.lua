local card = {
    api_version = 1, id = "ICC_809", name = "Plague Scientist",
    text = "<b>Combo:</b> Give a friendly minion <b>Poisonous</b>.", set = "ICECROWN",
    type = "minion", class = "rogue", rarity = "common", cost = 3, attack = 2, health = 3,
    tags = { "undead" }, keywords = { "combo" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        if not ctx:combo_active(self) then return {} end
        return ctx:friendly_minions(self)
    end,
}

function card.on_combo(ctx, self, target)
    if target ~= nil then ctx:grant_keyword(target, "poisonous") end
end

return card
