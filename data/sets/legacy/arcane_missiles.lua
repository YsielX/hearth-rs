local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "EX1_277",
    name = "Arcane Missiles",
    text = "Deal $3 damage randomly split among all enemies.",
    set = "LEGACY",
    type = "spell",
    class = "mage",
    rarity = "free",
    spell_school = "arcane",
    cost = 1,
}

local function choose_target(ctx, self, hook)
    local pool = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        if not is_dormant(ctx, enemy) then pool[#pool + 1] = enemy end
    end
    if #pool > 0 then ctx:random_entity(pool, hook) end
end

function card.on_play(ctx, self)
    local count = 3
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        count = count + ctx:entity(minion).spell_damage
    end
    ctx:set_data(self, "missiles_remaining", count)
    ctx:continue_with("fire_missile")
end

function card.fire_missile(ctx, self)
    if ctx:get_data(self, "missiles_remaining") > 0 then
        choose_target(ctx, self, "hit_with_missile")
    end
end

function card.hit_with_missile(ctx, self, target)
    ctx:damage(target, 1)
    local remaining = ctx:get_data(self, "missiles_remaining") - 1
    ctx:set_data(self, "missiles_remaining", remaining)
    if remaining > 0 then ctx:continue_with("fire_missile") end
end

card.triggers = {
    {
        event = "damaged", timing = "before", active_zones = { "graveyard" },
        condition = function(ctx, self, event) return event.source == self end,
        effect = function(ctx, self, event) ctx:set_event_amount(event, 1) end,
    },
}

return card
