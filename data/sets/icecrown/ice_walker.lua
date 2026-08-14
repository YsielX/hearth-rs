return {
    api_version = 1, id = "ICC_068", name = "Ice Walker",
    text = "Your Hero Power also <b><b>Freeze</b>s</b> the target.",
    set = "ICECROWN", type = "minion", class = "mage", rarity = "rare",
    cost = 2, attack = 1, health = 3, tags = { "elemental" },
    triggers = {{
        event = "hero_power_used", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.target ~= nil
        end,
        effect = function(ctx, self, event)
            local target = ctx:entity(event.target)
            if target.type == "hero" or (target.type == "minion" and target.zone == "board") then
                ctx:freeze(event.target)
            end
        end,
    }},
}
