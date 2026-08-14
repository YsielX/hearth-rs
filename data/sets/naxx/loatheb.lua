local card = {
    api_version = 1,
    id = "FP1_030",
    name = "Loatheb",
    text = "<b>Battlecry:</b> Enemy spells cost (5) more next turn.",
    set = "NAXX",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:set_player_data(player, "loatheb:" .. self, ctx:turn() + 1)
end

card.auras = {
    {
        active_zones = { "board", "graveyard", "hand", "deck", "removed" },
        cost = 5,
        targets = function(ctx, self)
            local player = ctx:controller(self)
            local expires = ctx:get_player_data(player, "loatheb:" .. self)
            if expires == 0 or ctx:turn() > expires then
                return {}
            end
            local enemy = ctx:opponent(player)
            local spells = {}
            for _, entity in ipairs(ctx:hand(enemy)) do
                if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
            end
            for _, entity in ipairs(ctx:deck(enemy)) do
                if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
            end
            return spells
        end,
    },
}

card.triggers = {
    {
        event = "turn_ended",
        active_zones = { "board", "graveyard", "hand", "deck", "removed" },
        condition = function(ctx, self, event)
            local player = ctx:controller(self)
            local expires = ctx:get_player_data(player, "loatheb:" .. self)
            return expires > 0
                and event.player == ctx:opponent(player)
                and event.turn >= expires
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            ctx:set_player_data(player, "loatheb:" .. self, 0)
        end,
    },
}

return card
