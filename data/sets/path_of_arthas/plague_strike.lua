local card = {
    api_version = 1,
    id = "RLK_018",
    name = "Plague Strike",
    text = "[x]Deal $3 damage\nto a minion. If it dies,\nsummon a 2/2 Zombie\nwith <b>Rush</b>.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "shadow",
    cost = 2,
    rune_cost = { unholy = 1 },
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 3)
    ctx:continue_with_entity("check_kill", target)
end

function card.check_kill(ctx, self, target)
    if ctx:entity(target).zone == "graveyard" then
        ctx:summon(ctx:controller(self), "RLK_018t")
    end
end

card.tokens = {{
    id = "RLK_018t",
    name = "Rampaging Zombie",
    text = "<b>Rush</b>",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "neutral",
    collectible = false,
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "undead" },
    keywords = { "rush" },
}}

return card
