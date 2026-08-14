return {
    api_version = 1,
    id = "AT_031",
    name = "Cutpurse",
    text = "Whenever this minion attacks a hero, add the Coin to your hand.",
    set = "TGT",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "undead" },
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.attacker == self and ctx:entity(event.defender).type == "hero"
            end,
            effect = function(ctx, self) ctx:give_card(ctx:controller(self), "GAME_005") end,
        },
    },
}
