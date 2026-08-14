local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_020",
    name = "Fel Cannon",
    text = "At the end of your turn, deal 2 damage to a non-Mech minion.",
    set = "GVG",
    type = "minion",
    class = "warlock",
    rarity = "rare",
    cost = 4,
    attack = 3,
    health = 5,
    tags = { "mech" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local candidates = {}
                for _, minion in ipairs(ctx:minions()) do
                    if not is_mech(ctx, minion) then candidates[#candidates + 1] = minion end
                end
                if #candidates > 0 then ctx:random_entity(candidates, "fire_cannon") end
            end,
        },
    },
}

function card.fire_cannon(ctx, self, target)
    ctx:damage(target, 2)
end

return card
