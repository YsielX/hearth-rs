local card = {
    api_version = 1,
    id = "RLK_225",
    name = "Blightfang",
    text = "[x]<b>Battlecry:</b> Infect all enemy\nminions. When they die,\nyou summon a 2/2\nZombie with <b>Taunt</b>.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 3,
    attack = 3,
    health = 3,
    rune_cost = { unholy = 1 },
    tags = { "beast" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        ctx:attach_hook(minion, "on_deathrattle", "RLK_225")
        cardlib.effects.grant_keyword(ctx, minion, "deathrattle")
    end
end

function card.on_deathrattle(ctx, self)
    ctx:summon(ctx:opponent(ctx:controller(self)), "RLK_118t3")
end

return card
