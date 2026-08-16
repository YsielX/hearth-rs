local function mine(ctx, self, e)
    local source = e.source; if not source then return false end; local x = ctx:entity(source); return x.controller ==
        ctx:controller(self) and (x.type == "spell" or x.type == "hero_power")
end
return {
    api_version = 1,
    id = "EX1_350",
    name = "Prophet Velen",
    text =
    "Double the damage and healing of your spells and Hero Power.",
    set = "EXPERT1",
    type = "minion",
    class = "priest",
    rarity =
    "legendary",
    cost = 7,
    attack = 7,
    health = 7,
    tags = { "draenei" },
    triggers = { {
        event = "damaged",
        timing = "before",
        active_zones = { "board" },
        condition = mine,
        effect = function(
            ctx, self, e)
            cardlib.effects.multiply_event_amount(ctx, e, 2)
        end
    }, {
        event = "healed",
        timing = "before",
        active_zones = { "board" },
        condition = mine,
        effect = function(
            ctx, self, e)
            cardlib.effects.multiply_event_amount(ctx, e, 2)
        end
    } }
}
