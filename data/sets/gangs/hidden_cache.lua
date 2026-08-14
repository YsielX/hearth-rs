local function hand_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1,
    id = "CFM_026",
    name = "Hidden Cache",
    text = "<b>Secret:</b> After your opponent plays a minion, give a random minion in your hand +2/+2.",
    set = "GANGS",
    type = "spell",
    class = "hunter",
    rarity = "rare",
    cost = 2,
    keywords = { "secret" },
    triggers = {{
        event = "minion_played", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:opponent(ctx:controller(self))
                and #hand_minions(ctx, ctx:controller(self)) > 0
        end,
        effect = function(ctx, self)
            ctx:reveal_secret(self)
            ctx:continue_with("choose_hand_minion")
        end,
    }},
}

function card.choose_hand_minion(ctx, self)
    local candidates = hand_minions(ctx, ctx:controller(self))
    if #candidates > 0 then ctx:random_entity(candidates, "buff_hand_minion") end
end

function card.buff_hand_minion(ctx, self, target) ctx:buff(target, 2, 2) end

return card
