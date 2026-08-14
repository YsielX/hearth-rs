local card = {
    api_version = 1,
    id = "ICC_832",
    name = "Malfurion the Pestilent",
    text = "[x]<b>Choose One -</b>\nSummon 2 <b>Poisonous</b>\nSpiders; or 2 Scarabs\nwith <b>Taunt</b>.",
    set = "ICECROWN",
    type = "hero",
    class = "druid",
    cost = 7,
    health = 30,
    armor = 5,
    hero_power = "ICC_832p",
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Summon two Scarab Beetles", value = 1 },
        { label = "Summon two Frost Widows", value = 2 },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    local token = choice == 1 and "ICC_832t4" or "ICC_832t3"
    local player = ctx:controller(self)
    ctx:summon(player, token)
    ctx:summon(player, token)
end

card.tokens = {
    {
        id = "ICC_832t3", name = "Frost Widow", text = "<b>Poisonous</b>",
        set = "ICECROWN", type = "minion", class = "druid",
        cost = 1, attack = 1, health = 2, tags = { "beast" }, keywords = { "poisonous" },
    },
    {
        id = "ICC_832t4", name = "Scarab Beetle", text = "<b>Taunt</b>",
        set = "ICECROWN", type = "minion", class = "druid",
        cost = 2, attack = 1, health = 5, tags = { "beast" }, keywords = { "taunt" },
    },
}

return card
