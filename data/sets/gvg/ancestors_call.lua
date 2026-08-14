local function hand_minions(ctx, player)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).type == "minion" then
            candidates[#candidates + 1] = entity
        end
    end
    return candidates
end

local card = {
    api_version = 1,
    id = "GVG_029",
    name = "Ancestor's Call",
    text = "Put a random minion from each player's hand into the battlefield.",
    set = "GVG",
    type = "spell",
    class = "shaman",
    rarity = "epic",
    cost = 4,
}

function card.on_play(ctx, self)
    local candidates = hand_minions(ctx, ctx:controller(self))
    if #candidates > 0 then
        ctx:random_entity(candidates, "summon_friendly_minion")
    else
        ctx:continue_with("choose_enemy_minion")
    end
end

function card.summon_friendly_minion(ctx, self, entity)
    ctx:summon_from_hand(entity)
    ctx:continue_with("choose_enemy_minion")
end

function card.choose_enemy_minion(ctx, self)
    local candidates = hand_minions(ctx, ctx:opponent(ctx:controller(self)))
    if #candidates > 0 then ctx:random_entity(candidates, "summon_enemy_minion") end
end

function card.summon_enemy_minion(ctx, self, entity)
    ctx:summon_from_hand(entity)
end

return card
