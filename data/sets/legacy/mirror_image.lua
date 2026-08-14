return {
    api_version = 1,
    id = "CS2_027",
    name = "Mirror Image",
    text = "Summon two 0/2 minions with <b>Taunt</b>.",
    set = "LEGACY",
    type = "spell",
    class = "mage",
    rarity = "free",
    cost = 1,
    rules = {
        can_play = function(ctx, self)
            return #ctx:friendly_minions(self) < 7
        end,
    },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "CS2_mirror")
        ctx:summon(player, "CS2_mirror")
    end,
    tokens = {
        {
            id = "CS2_mirror",
            name = "Mirror Image",
            text = "<b>Taunt</b>",
            set = "LEGACY",
            type = "minion",
            class = "mage",
            rarity = "common",
            cost = 0,
            attack = 0,
            health = 2,
            keywords = { "taunt" },
        },
    },
}
