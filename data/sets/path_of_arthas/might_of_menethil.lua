local card = {
    api_version = 1,
    id = "RLK_740",
    name = "Might of Menethil",
    text = "<b>Battlecry:</b> Spend\nup to 3 <b>Corpses</b>.\n<b>Freeze</b> that many\nenemy minions.",
    set = "PATH_OF_ARTHAS",
    type = "weapon",
    class = "death_knight",
    rarity = "epic",
    cost = 4,
    attack = 4,
    health = 2,
    rune_cost = { frost = 2 },
    keywords = { "battlecry" },
}

local function remaining_minions(ctx, self)
    local candidates = {}
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        if ctx:get_data(self, "menethil_frozen:" .. minion) == 0 then
            candidates[#candidates + 1] = minion
        end
    end
    return candidates
end

function card.on_battlecry(ctx, self)
    local candidates = remaining_minions(ctx, self)
    local maximum = math.min(3, #candidates)
    if maximum > 0 then
        ctx:spend_resource_and_continue(
            ctx:controller(self), "corpses", 1, maximum, "menethil_paid"
        )
    end
end

function card.menethil_paid(ctx, self, spent)
    ctx:set_data(self, "menethil_freezes_left", spent)
    ctx:continue_with("menethil_choose")
end

function card.menethil_choose(ctx, self)
    if ctx:get_data(self, "menethil_freezes_left") <= 0 then return end
    local candidates = remaining_minions(ctx, self)
    if #candidates > 0 then ctx:random_entity(candidates, "menethil_freeze") end
end

function card.menethil_freeze(ctx, self, target)
    ctx:freeze(target)
    ctx:set_data(self, "menethil_frozen:" .. target, 1)
    local left = ctx:get_data(self, "menethil_freezes_left") - 1
    ctx:set_data(self, "menethil_freezes_left", left)
    if left > 0 then ctx:continue_with("menethil_choose") end
end

return card
