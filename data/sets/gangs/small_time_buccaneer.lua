return {
    api_version = 1,
    id = "CFM_325",
    name = "Small-Time Buccaneer",
    text = "Has +2 Attack while you have a weapon equipped.",
    set = "GANGS",
    type = "minion",
    rarity = "rare",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "pirate" },
    auras = {{
        active_zones = { "board" },
        attack = function(ctx, self)
            return ctx:player(ctx:controller(self)).weapon and 2 or 0
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
