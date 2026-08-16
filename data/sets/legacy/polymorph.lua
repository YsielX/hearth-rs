return {
    api_version = 1,
    id = "CS2_022",
    name = "Polymorph",
    text = "Transform a minion\ninto a 1/1 Sheep.",
    set = "LEGACY",
    type = "spell",
    class = "mage",
    cost = 4,
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:minions()
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.transform(ctx, target, "CS2_tk1")
    end,
    tokens = {
        {
            id = "CS2_tk1",
            name = "Sheep",
            text = "",
            set = "LEGACY",
            type = "minion",
            class = "mage",
            cost = 1,
            attack = 1,
            health = 1,
            tags = { "beast" },
        },
    },
}
