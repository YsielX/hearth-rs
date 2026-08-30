local function is_undead(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "RLK_057",
    name = "Dark Transformation",
    text = "Transform an Undead into a 4/5 Undead Monstrosity with <b>Rush</b>.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "shadow",
    cost = 2,
    rune_cost = { unholy = 1 },
    target_mode = "required",
    targets = function(ctx)
        local targets = {}
        for _, entity in ipairs(ctx:minions()) do
            if is_undead(ctx, entity) then targets[#targets + 1] = entity end
        end
        return targets
    end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.transform(ctx, target, "RLK_057t")
end

card.tokens = {{
    id = "RLK_057t",
    name = "Undead Monstrosity",
    text = "<b>Rush</b>",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 4,
    attack = 4,
    health = 5,
    tags = { "undead" },
    keywords = { "rush" },
}}

return card
