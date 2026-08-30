local card = {
    api_version = 1, id = "CFM_631", name = "Brass Knuckles",
    text = "[x]After your hero attacks,\ngive a random minion in\nyour hand +1/+1.",
    set = "GANGS", type = "weapon", class = "warrior", rarity = "epic",
    cost = 3, attack = 2, health = 3,
    triggers = {{
        event = "attack", timing = "after", active_zones = { "weapon" },
        condition = function(ctx, self, event)
            return event.attacker == ctx:player(ctx:controller(self)).hero
        end,
        effect = function(ctx, self)
            local candidates = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
            end
            if #candidates > 0 then ctx:random_entity(candidates, "buff_minion") end
        end,
    }},
}
function card.buff_minion(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end
return card
