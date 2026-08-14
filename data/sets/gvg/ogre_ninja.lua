local card = {
    api_version = 1,
    id = "GVG_088",
    name = "Ogre Ninja",
    text = "<b>Stealth</b>\n50% chance to attack the wrong enemy.",
    set = "GVG",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 5,
    attack = 6,
    health = 6,
    keywords = { "stealth" },
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.attacker == self
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "attack_event", event.event_id)
                ctx:set_data(self, "chosen_defender", event.defender)
                ctx:random_value({ 0, 1 }, "roll_wrong_attack")
            end,
        },
    },
}

function card.roll_wrong_attack(ctx, self, roll)
    if roll == 0 then return end
    local chosen = ctx:get_data(self, "chosen_defender")
    local candidates = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        if enemy ~= chosen then candidates[#candidates + 1] = enemy end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "redirect_attack") end
end

function card.redirect_attack(ctx, self, defender)
    ctx:set_attack_defender(ctx:get_data(self, "attack_event"), defender)
end

return card
