local card = {
    api_version = 1, id = "AT_088", name = "Mogor's Champion",
    text = "50% chance to attack the wrong enemy.",
    set = "TGT", type = "minion", rarity = "rare", cost = 6, attack = 8, health = 5,
}

local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

card.triggers = {{
    event = "attack", timing = "before", active_zones = { "board" },
    condition = function(ctx, self, event) return event.attacker == self end,
    effect = function(ctx, self, event)
        ctx:set_data(self, "attack_event", event.event_id)
        ctx:set_data(self, "declared_defender", event.defender)
        ctx:random_value({ 0, 1 }, "roll_wrong_attack")
    end,
}}

function card.roll_wrong_attack(ctx, self, wrong)
    if wrong == 0 then return end
    local candidates = {}
    local declared = ctx:get_data(self, "declared_defender")
    for _, target in ipairs(ctx:enemy_characters(self)) do
        if target ~= declared and not dormant(ctx, target) then
            candidates[#candidates + 1] = target
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "redirect_attack") end
end

function card.redirect_attack(ctx, self, target)
    ctx:set_attack_defender(ctx:get_data(self, "attack_event"), target)
end

return card
