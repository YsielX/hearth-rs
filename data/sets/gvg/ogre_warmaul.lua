local card = {
    api_version = 1,
    id = "GVG_054",
    name = "Ogre Warmaul",
    text = "50% chance to attack the wrong enemy.",
    set = "GVG",
    type = "weapon",
    class = "warrior",
    rarity = "common",
    cost = 3,
    attack = 4,
    health = 2,
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                local player = ctx:controller(self)
                return event.attacker == ctx:player(player).hero
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "attack_event", event.event_id)
                ctx:set_data(self, "chosen_defender", event.defender)
                ctx:random_value({ 0, 1 }, "roll_wrong_enemy")
            end,
        },
    },
}

function card.roll_wrong_enemy(ctx, self, roll)
    if roll == 0 then return end
    local chosen = ctx:get_data(self, "chosen_defender")
    local alternatives = {}
    for _, defender in ipairs(ctx:enemy_characters(self)) do
        if defender ~= chosen then alternatives[#alternatives + 1] = defender end
    end
    if #alternatives > 0 then ctx:random_entity(alternatives, "redirect_attack") end
end

function card.redirect_attack(ctx, self, defender)
    ctx:set_attack_defender(ctx:get_data(self, "attack_event"), defender)
end

return card
