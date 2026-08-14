return {
    api_version = 1,
    id = "TOY_811",
    name = "Tigress Plushy",
    text = "<b>Miniaturize</b>\n<b>Rush</b>, <b>Lifesteal</b>,\n<b>Divine Shield</b>",
    set = "WHIZBANGS_WORKSHOP",
    type = "minion",
    class = "paladin",
    tags = { "beast" },
    cost = 3,
    attack = 3,
    health = 2,
    keywords = { "miniaturize", "rush", "lifesteal", "divine_shield" },
    on_miniaturize = function(ctx, self)
        ctx:give_card(ctx:controller(self), "TOY_811t")
    end,
    tokens = {
        {
            id = "TOY_811t",
            name = "Tigress Plushy",
            text = "<b>Mini</b>\n<b>Rush</b>, <b>Lifesteal</b>,\n<b>Divine Shield</b>",
            set = "WHIZBANGS_WORKSHOP",
            type = "minion",
            class = "paladin",
            tags = { "beast" },
            cost = 1,
            attack = 1,
            health = 1,
            keywords = { "rush", "lifesteal", "divine_shield" },
        },
    },
}
