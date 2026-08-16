local card = {
    api_version = 1,
    id = "EX1_298",
    name = "Ragnaros the Firelord",
    text = "Can't attack. At the end of your turn, deal 8 damage to a random enemy.",
    set = "EXPERT1",
    type = "minion",
    rarity = "legendary",
    cost = 8,
    attack = 8,
    health = 8,
    tags = { "elemental" },
    rules = {
        can_attack = function(ctx, self, current)
            return false
        end,
    },
    triggers = { {
        event = "turn_ended",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:random_entity(ctx:enemy_characters(self), "firelord_hit")
        end,
    } },
}

function card.firelord_hit(ctx, self, target)
    cardlib.effects.damage(ctx, target, 8)
end

return card
