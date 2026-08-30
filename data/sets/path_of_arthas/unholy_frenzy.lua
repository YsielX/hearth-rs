local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "RLK_056",
    name = "Unholy Frenzy",
    text = "[x]Choose an enemy minion.\nYour minions attack it.\nResummon any that die.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    rune_cost = { unholy = 1 },
    target_mode = "required",
    targets = function(ctx, self) return ctx:enemy_minions(self) end,
}

function card.on_play(ctx, self, target)
    local attackers = ctx:friendly_minions(self)
    ctx:set_data(self, "frenzy_target", target)
    ctx:set_data(self, "frenzy_count", #attackers)
    ctx:set_data(self, "frenzy_next", 1)
    ctx:set_data(self, "frenzy_armed", 1)
    for index, attacker in ipairs(attackers) do
        ctx:set_data(self, "frenzy_attacker_" .. index, attacker)
        ctx:set_data(self, "frenzy_position_" .. index, index - 1)
    end
    ctx:continue_with("frenzy_attack_next")
end

function card.frenzy_attack_next(ctx, self)
    local target = ctx:get_data(self, "frenzy_target")
    local index = ctx:get_data(self, "frenzy_next")
    local count = ctx:get_data(self, "frenzy_count")
    if index > count or ctx:entity(target).zone ~= "board" then
        ctx:set_data(self, "frenzy_armed", 0)
        ctx:set_data(self, "frenzy_resummon_next", 1)
        ctx:continue_with("frenzy_resummon_next")
        return
    end

    ctx:set_data(self, "frenzy_next", index + 1)
    local attacker = ctx:get_data(self, "frenzy_attacker_" .. index)
    if ctx:entity(attacker).zone == "board" and not is_dormant(ctx, attacker) then
        ctx:force_attack(attacker, target)
        ctx:continue_with("frenzy_attack_next")
    else
        ctx:continue_with("frenzy_attack_next")
    end
end

function card.frenzy_resummon_next(ctx, self)
    local index = ctx:get_data(self, "frenzy_resummon_next")
    local count = ctx:get_data(self, "frenzy_count")
    while index <= count and ctx:get_data(self, "frenzy_died_" .. index) == 0 do
        index = index + 1
    end
    if index > count then return end

    ctx:set_data(self, "frenzy_resummon_next", index + 1)
    local entity = ctx:get_data(self, "frenzy_attacker_" .. index)
    local position = ctx:get_data(self, "frenzy_position_" .. index)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    cardlib.effects.summon_fresh_copy(ctx, entity, position, definition.health, {})
    ctx:continue_with("frenzy_resummon_next")
end

card.triggers = {{
    event = "entity_died",
    timing = "after",
    active_zones = { "graveyard" },
    condition = function(ctx, self, event)
        if ctx:get_data(self, "frenzy_armed") ~= 1 then return false end
        for index = 1, ctx:get_data(self, "frenzy_count") do
            if ctx:get_data(self, "frenzy_attacker_" .. index) == event.entity then return true end
        end
        return false
    end,
    effect = function(ctx, self, event)
        for index = 1, ctx:get_data(self, "frenzy_count") do
            if ctx:get_data(self, "frenzy_attacker_" .. index) == event.entity then
                ctx:set_data(self, "frenzy_died_" .. index, 1)
                return
            end
        end
    end,
}}

return card
