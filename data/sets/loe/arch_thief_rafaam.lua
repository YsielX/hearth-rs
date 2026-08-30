local function damageable_enemies(ctx, self)
    local result = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(enemy).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then result[#result + 1] = enemy end
    end
    return result
end

local card = {
    api_version = 1,
    id = "LOE_092",
    name = "Arch-Thief Rafaam",
    text = "<b>Battlecry: Discover</b> a powerful Artifact.",
    set = "LOE",
    type = "minion",
    rarity = "legendary",
    cost = 9,
    attack = 7,
    health = 8,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local prompt = ctx:localize(
        "Discover a powerful Artifact",
        "发现一件强大的神器",
        "發現一件強大的神器"
    )
    ctx:discover_cards(
        ctx:controller(self),
        prompt,
        { "LOEA16_3", "LOEA16_4", "LOEA16_5" },
        3,
        "on_artifact_discovered"
    )
end

function card.on_artifact_discovered(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

card.tokens = {
    {
        id = "LOEA16_3",
        name = "Lantern of Power",
        text = "Give a minion +10/+10.",
        set = "LOE",
        type = "spell",
        cost = 10,
        target_mode = "required",
        targets = function(ctx) return ctx:minions() end,
        on_play = function(ctx, self, target) cardlib.effects.buff(ctx, target, 10, 10) end,
    },
    {
        id = "LOEA16_4",
        name = "Timepiece of Horror",
        text = "Deal $10 damage randomly split among all enemies.",
        set = "LOE",
        type = "spell",
        cost = 10,
        on_play = function(ctx, self)
            ctx:set_data(self, "horror_damage_left", 10)
            ctx:continue_with("choose_horror_target")
        end,
        choose_horror_target = function(ctx, self)
            local remaining = ctx:get_data(self, "horror_damage_left") or 0
            if remaining > 0 then
                local candidates = damageable_enemies(ctx, self)
                if #candidates > 0 then
                    ctx:random_entity(candidates, "deal_horror_damage")
                end
            end
        end,
        deal_horror_damage = function(ctx, self, target)
            local remaining = ctx:get_data(self, "horror_damage_left") or 0
            if remaining <= 0 then return end
            ctx:set_data(self, "horror_damage_left", remaining - 1)
            cardlib.effects.damage(ctx, target, 1)
            if remaining > 1 then ctx:continue_with("choose_horror_target") end
        end,
    },
    {
        id = "LOEA16_5",
        name = "Mirror of Doom",
        text = "Fill your board with 3/3 Mummy Zombies.",
        set = "LOE",
        type = "spell",
        cost = 10,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            local available = 7 - #ctx:board(player)
            for _ = 1, available do ctx:summon(player, "LOEA16_5t") end
        end,
    },
    {
        id = "LOEA16_5t",
        name = "Mummy Zombie",
        text = "",
        set = "LOE",
        type = "minion",
        cost = 3,
        attack = 3,
        health = 3,
        tags = { "undead" },
    },
}

return card
