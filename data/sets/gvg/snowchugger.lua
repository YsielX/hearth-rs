return {
    api_version = 1,
    id = "GVG_002",
    name = "Snowchugger",
    text = "<b>Freeze</b> any character damaged by this minion.",
    set = "GVG",
    type = "minion",
    class = "mage",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "mech" },
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.source == self and event.amount > 0
            end,
            effect = function(ctx, self, event)
                ctx:freeze(event.target)
            end,
        },
    },
}
