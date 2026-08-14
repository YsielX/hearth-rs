local function other_friendly_mechs(ctx, self)
    local candidates = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then
            local definition = ctx:card_definition(ctx:entity(minion).card_id)
            for _, tag in ipairs(definition.tags) do
                if tag == "mech" then candidates[#candidates + 1] = minion break end
            end
        end
    end
    return candidates
end

local card = {
    api_version = 1,
    id = "GVG_027",
    name = "Iron Sensei",
    text = "At the end of your turn, give another friendly Mech +2/+2.",
    set = "GVG",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 3,
    attack = 2,
    health = 2,
    tags = { "mech" },
    triggers = {
        {
            event = "turn_ended",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and #other_friendly_mechs(ctx, self) > 0
            end,
            effect = function(ctx, self)
                ctx:random_entity(other_friendly_mechs(ctx, self), "upgrade_mech")
            end,
        },
    },
}

function card.upgrade_mech(ctx, self, target)
    ctx:buff(target, 2, 2)
end

return card
