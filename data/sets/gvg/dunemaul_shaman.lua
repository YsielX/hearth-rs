local card = {
    api_version = 1,
    id = "GVG_066",
    name = "Dunemaul Shaman",
    text = "<b>Windfury, Overload:</b> (1)\n50% chance to attack the wrong enemy.",
    set = "GVG",
    type = "minion",
    class = "shaman",
    rarity = "rare",
    cost = 4,
    attack = 5,
    health = 4,
    keywords = { "windfury", "overload" },
    keyword_params = { overload = 1 },
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
