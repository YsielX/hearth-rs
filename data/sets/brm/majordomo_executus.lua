local card = {
    api_version = 1,
    id = "BRM_027",
    name = "Majordomo Executus",
    text = "<b>Deathrattle:</b> Replace your hero with Ragnaros the Firelord.",
    set = "BRM",
    type = "minion",
    rarity = "legendary",
    cost = 9,
    attack = 9,
    health = 7,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    ctx:replace_hero(ctx:controller(self), "BRM_027h")
end

card.tokens = {
    {
        id = "BRM_027h",
        name = "Ragnaros the Firelord",
        text = "",
        set = "BRM",
        type = "hero",
        cost = 0,
        health = 8,
        hero_power = "BRM_027p",
    },
    {
        id = "BRM_027p",
        name = "DIE, INSECT!",
        text = "Deal $8 damage to a random enemy.",
        set = "BRM",
        type = "hero_power",
        cost = 2,
        on_play = function(ctx, self)
            ctx:random_entity(ctx:enemy_characters(self), "deal_ragnaros_damage")
        end,
        deal_ragnaros_damage = function(ctx, self, target)
            cardlib.effects.damage(ctx, target, 8)
        end,
    },
    {
        id = "BRM_027pH",
        name = "DIE, INSECTS!",
        text = "Deal $8 damage to a\nrandom enemy. TWICE.",
        set = "BRM",
        type = "hero_power",
        cost = 2,
        on_play = function(ctx, self)
            ctx:random_entity(ctx:enemy_characters(self), "deal_first_ragnaros_damage")
        end,
        deal_first_ragnaros_damage = function(ctx, self, target)
            cardlib.effects.damage(ctx, target, 8)
            ctx:continue_with("choose_second_ragnaros_target")
        end,
        choose_second_ragnaros_target = function(ctx, self)
            ctx:random_entity(ctx:enemy_characters(self), "deal_second_ragnaros_damage")
        end,
        deal_second_ragnaros_damage = function(ctx, self, target)
            cardlib.effects.damage(ctx, target, 8)
        end,
    },
}

return card
