local card = {
    api_version = 1,
    id = "GVG_072",
    name = "Shadowboxer",
    text = "Whenever a minion is healed, deal 1 damage to a random enemy.",
    set = "GVG",
    type = "minion",
    class = "priest",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "mech" },
    triggers = {
        {
            event = "healed",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.amount > 0 and ctx:entity(event.target).type == "minion"
            end,
            effect = function(ctx, self)
                local enemies = ctx:enemy_characters(self)
                if #enemies > 0 then ctx:random_entity(enemies, "punch") end
            end,
        },
    },
}

function card.punch(ctx, self, target)
    ctx:damage(target, 1)
end

return card
