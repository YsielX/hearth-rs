local function death_count(ctx, self)
    local player = ctx:controller(self)
    return #ctx:minions_died_this_turn(player)
        + #ctx:minions_died_this_turn(ctx:opponent(player))
end

local function candidates(ctx)
    local result = {}
    for _, minion in ipairs(ctx:minions()) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then result[#result + 1] = minion end
    end
    return result
end

local function choose_target(ctx, self)
    local pool = candidates(ctx)
    if #pool > 0 then ctx:random_entity(pool, "hit_random_minion") end
end

local card = {
    api_version = 1,
    id = "GVG_050",
    name = "Bouncing Blade",
    text = "Deal $1 damage to a random minion. Repeat until a minion dies.",
    set = "GVG",
    type = "spell",
    class = "warrior",
    rarity = "epic",
    cost = 2,
    rules = {
        can_play = function(ctx, self, current)
            return current and #candidates(ctx) > 0
        end,
    },
}

function card.on_play(ctx, self)
    ctx:set_data(self, "starting_deaths", death_count(ctx, self))
    choose_target(ctx, self)
end

function card.hit_random_minion(ctx, self, target)
    cardlib.effects.damage(ctx, target, 1)
    ctx:continue_with("continue_bouncing")
end

function card.continue_bouncing(ctx, self)
    if death_count(ctx, self) == ctx:get_data(self, "starting_deaths") then
        choose_target(ctx, self)
    end
end

return card
