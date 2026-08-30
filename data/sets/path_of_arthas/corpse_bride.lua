local card = {
    api_version = 1,
    id = "RLK_504",
    name = "Corpse Bride",
    text = "[x]<b>Battlecry:</b> Spend up to 10\n <b>Corpses</b> to summon a Risen \nGroom with <b>Taunt</b> and that\n much Attack and Health.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 5,
    attack = 4,
    health = 4,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end
    local spent = ctx:spend_up_to_corpses(player, 10)
    ctx:summon_with_stats(player, "RLK_506t", spent, spent)
end

card.tokens = {{
    id = "RLK_506t",
    name = "Risen Groom",
    text = "<b>Taunt</b>\n<i>Doesn't leave a <b>Corpse</b>.</i>",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "undead" },
    keywords = { "taunt", "no_corpse" },
}}

return card
