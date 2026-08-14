return {
    api_version = 1,
    id = "GVG_051",
    name = "Warbot",
    text = "Has +1 Attack while damaged.",
    set = "GVG",
    type = "minion",
    class = "warrior",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 3,
    tags = { "mech" },
    auras = {
        {
            attack = function(ctx, self)
                if ctx:entity(self).damage > 0 then return 1 end
                return 0
            end,
            targets = function(ctx, self) return { self } end,
        },
    },
}
