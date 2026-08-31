local card = {
    api_version = 1,
    id = "ICC_832",
    name = "Malfurion the Pestilent",
    text = "[x]<b>Choose One -</b>\nSummon 2 <b>Poisonous</b>\nSpiders; or 2 Scarabs\nwith <b>Taunt</b>.",
    set = "ICECROWN",
    type = "hero",
    class = "druid",
    rarity = "legendary",
    cost = 7,
    health = 30,
    armor = 5,
    hero_power = "ICC_832p",
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "ICC_832a", label = "Summon two Scarab Beetles" },
        { card_id = "ICC_832b", label = "Summon two Frost Widows" },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    local token = choice == "ICC_832a" and "ICC_832t4" or "ICC_832t3"
    local player = ctx:controller(self)
    ctx:summon(player, token)
    ctx:summon(player, token)
end

function card.on_choose_multiple(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "ICC_832t3")
    ctx:summon(player, "ICC_832t3")
    ctx:summon(player, "ICC_832t4")
    ctx:summon(player, "ICC_832t4")
end

card.tokens = {
    {
        id = "ICC_832a", name = "Scarab Plague", text = "Summon two 1/5 Scarabs with <b>Taunt</b>.",
        set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 7,
    },
    {
        id = "ICC_832b", name = "Spider Plague", text = "[x]Summon two 1/2\nSpiders with <b>Poisonous</b>.",
        set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 7,
    },
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
