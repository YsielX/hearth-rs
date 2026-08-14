local card = {
    api_version = 1, id = "ICC_240", name = "Runeforge Haunter",
    text = "During your turn, your weapon doesn't lose Durability.", set = "ICECROWN",
    type = "minion", class = "rogue", rarity = "rare", cost = 4, attack = 5, health = 3,
    tags = { "undead" },
}

card.auras = {{
    keywords = { "weapon_durability_immune" },
    targets = function(ctx, self)
        local player = ctx:controller(self)
        local weapon = ctx:player(player).weapon
        if ctx:active_player() == player and weapon ~= nil then return { weapon } end
        return {}
    end,
}}

return card
