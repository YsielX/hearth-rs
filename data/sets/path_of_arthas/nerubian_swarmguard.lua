local card = {
    api_version = 1,
    id = "RLK_062",
    name = "Nerubian Swarmguard",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Summon two\ncopies of this minion.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 4,
    attack = 1,
    health = 3,
    tags = { "undead" },
    keywords = { "taunt", "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:summon_copy(ctx:controller(self), self)
    ctx:summon_copy(ctx:controller(self), self)
end

return card
