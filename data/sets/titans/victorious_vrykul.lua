return {
    api_version = 1,
    id = "TTN_812",
    name = "Victorious Vrykul",
    text = "After this attacks, get a 2/3 Val'kyr that costs (1).",
    set = "TITANS",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 2,
    triggers = {
        {
            event = "attack",
            active_zones = { "board" },
            condition = function(ctx, self, event) return event.attacker == self end,
            effect = function(ctx, self, event)
                ctx:give_card(ctx:controller(self), "TTN_812t")
            end,
        },
    },
    tokens = {
        {
            id = "TTN_812t",
            name = "Victorious Val'kyr",
            text = "",
            set = "TITANS",
            type = "minion",
            cost = 1,
            attack = 2,
            health = 3,
            tags = { "undead" },
        },
    },
}
