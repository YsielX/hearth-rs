local card = {
    api_version = 1,
    id = "BRM_002",
    name = "Flamewaker",
    text = "After you cast a spell, deal 2 damage randomly split among all enemies.",
    set = "BRM",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 3,
    attack = 2,
    health = 4,
}

local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local function random_enemy(ctx, self, hook)
    local enemies = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        if not is_dormant(ctx, enemy) then enemies[#enemies + 1] = enemy end
    end
    if #enemies > 0 then ctx:random_entity(enemies, hook) end
end

card.triggers = {
    {
        event = "spell_cast",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self)
            ctx:continue_with("fire_first_shot")
        end,
    },
}

function card.fire_first_shot(ctx, self)
    random_enemy(ctx, self, "deal_first_damage")
end

function card.deal_first_damage(ctx, self, target)
    cardlib.effects.damage(ctx, target, 1)
    ctx:continue_with("fire_second_shot")
end

function card.fire_second_shot(ctx, self)
    random_enemy(ctx, self, "deal_second_damage")
end

function card.deal_second_damage(ctx, self, target)
    cardlib.effects.damage(ctx, target, 1)
end

return card
