local card = {
    api_version = 1, id = "GVG_065", name = "Ogre Brute",
    text = "50% chance to attack the wrong enemy.", set = "GVG", type = "minion",
    rarity = "common", cost = 3, attack = 4, health = 4,
}
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
        if target ~= declared then candidates[#candidates + 1] = target end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "redirect_attack") end
end
function card.redirect_attack(ctx, self, target)
    ctx:set_attack_defender(ctx:get_data(self, "attack_event"), target)
end
return card
