local card = {
    api_version = 1,
    id = "RLK_118",
    name = "Tomb Guardians",
    text = "Summon two 2/2 Zombies with <b>Taunt</b>. Spend 4 <b>Corpses</b> to\ngive them <b>Reborn</b>.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 4,
    rune_cost = { unholy = 2 },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local summon_count = math.min(2, 7 - #ctx:board(player))
    if summon_count <= 0 then return end
    ctx:set_data(self, "tomb_guardian_count", summon_count)
    ctx:spend_resource_and_continue(player, "corpses", 4, 4, "summon_guardians")
end

function card.summon_guardians(ctx, self, spent)
    local player = ctx:controller(self)
    local summon_count = ctx:get_data(self, "tomb_guardian_count")
    for _ = 1, summon_count do
        if spent > 0 then
            cardlib.effects.summon_with_stats(ctx, player, "RLK_118t3", 2, 2, { "reborn" })
        else
            ctx:summon(player, "RLK_118t3")
        end
    end
end

card.tokens = {{
    id = "RLK_118t3",
    name = "Menacing Zombie",
    text = "<b>Taunt</b>",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "undead" },
    keywords = { "taunt" },
}}

return card
