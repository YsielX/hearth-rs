return {
    api_version = 1,
    id = "RLK_063",
    name = "Frostwyrm's Fury",
    text = "Deal $5 damage. <b>Freeze</b> all enemy minions.\nSummon a 5/5 Frostwyrm.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "epic",
    spell_school = "frost",
    cost = 7,
    rune_cost = { frost = 3 },
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:enemy_characters(self)
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 5)
        for _, minion in ipairs(ctx:enemy_minions(self)) do
            ctx:freeze(minion)
        end
        ctx:summon(ctx:controller(self), "RLK_063t")
    end,
    tokens = {
        {
            id = "RLK_063t",
            name = "Frostwyrm",
            text = "",
            set = "PATH_OF_ARTHAS",
            type = "minion",
            class = "death_knight",
            collectible = false,
            cost = 5,
            attack = 5,
            health = 5,
            tags = { "undead", "dragon" },
        },
    },
}
