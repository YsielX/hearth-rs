local card = {
    api_version = 1, id = "CFM_754", name = "Grimy Gadgeteer",
    text = "At the end of your turn, give a random minion in your hand +2/+2.",
    set = "GANGS", type = "minion", class = "warrior", rarity = "common",
    cost = 3, attack = 4, health = 3,
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            local candidates = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
            end
            if #candidates > 0 then ctx:random_entity(candidates, "buff_minion") end
        end,
    }},
}
function card.buff_minion(ctx, self, target) ctx:buff(target, 2, 2) end
return card
