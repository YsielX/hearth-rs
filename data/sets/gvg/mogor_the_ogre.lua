local card = {
    api_version = 1, id = "GVG_112", name = "Mogor the Ogre",
    text = "All minions have a 50% chance to attack the wrong enemy.", set = "GVG",
    type = "minion", rarity = "legendary", cost = 6, attack = 7, health = 6,
}
card.triggers = {{
    event = "attack", timing = "before", active_zones = { "board" },
    condition = function(ctx, self, event) return ctx:entity(event.attacker).type == "minion" end,
    effect = function(ctx, self, event)
        ctx:set_data(self, "attack_event", event.event_id)
        ctx:set_data(self, "declared_defender", event.defender)
        ctx:set_data(self, "attacker", event.attacker)
        ctx:random_value({ 0, 1 }, "roll_wrong_attack")
    end,
}}
function card.roll_wrong_attack(ctx, self, wrong)
    if wrong == 0 then return end
    local candidates = {}
    local attacker = ctx:get_data(self, "attacker")
    local declared = ctx:get_data(self, "declared_defender")
    for _, target in ipairs(ctx:enemy_characters(attacker)) do
        if target ~= declared then candidates[#candidates + 1] = target end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "redirect_attack") end
end
function card.redirect_attack(ctx, self, target)
    ctx:set_attack_defender(ctx:get_data(self, "attack_event"), target)
end
return card
