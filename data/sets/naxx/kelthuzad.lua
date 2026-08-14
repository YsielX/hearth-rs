return {
    api_version = 1,
    id = "FP1_013",
    name = "Kel'Thuzad",
    text = "At the end of each turn, summon all friendly minions that died this turn.",
    set = "NAXX",
    type = "minion",
    rarity = "legendary",
    cost = 8,
    attack = 6,
    health = 8,
    tags = { "undead" },
    triggers = {
        {
            event = "turn_ended",
            active_zones = { "board" },
            condition = function(ctx, self)
                return #ctx:minions_died_this_turn(ctx:controller(self)) > 0
            end,
            effect = function(ctx, self)
                local player = ctx:controller(self)
                for _, card_id in ipairs(ctx:minions_died_this_turn(player)) do
                    ctx:summon(player, card_id)
                end
            end,
        },
    },
}
