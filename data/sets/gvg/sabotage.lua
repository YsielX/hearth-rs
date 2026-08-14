local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_047",
    name = "Sabotage",
    text = "Destroy a random enemy minion. <b>Combo:</b> And your opponent's weapon.",
    set = "GVG",
    type = "spell",
    class = "rogue",
    rarity = "epic",
    cost = 4,
    keywords = { "combo" },
    rules = {
        can_play = function(ctx, self, current)
            if not current then return false end
            local opponent = ctx:opponent(ctx:controller(self))
            for _, entity in ipairs(ctx:board(opponent)) do
                if ctx:entity(entity).type == "minion" and not is_dormant(ctx, entity) then return true end
            end
            return ctx:cards_played_this_turn(ctx:controller(self)) > 0
                and ctx:player(opponent).weapon ~= nil
        end,
    },
}

function card.on_play(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local candidates = {}
    for _, entity in ipairs(ctx:board(opponent)) do
        if ctx:entity(entity).type == "minion" and not is_dormant(ctx, entity) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "destroy_minion") end
end

function card.destroy_minion(ctx, self, target)
    ctx:destroy(target)
end

function card.on_combo(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local weapon = ctx:player(opponent).weapon
    if weapon ~= nil then ctx:destroy(weapon) end
end

return card
