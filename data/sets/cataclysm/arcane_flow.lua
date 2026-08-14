local card = {
    api_version = 1,
    id = "CATA_489",
    name = "Arcane Flow",
    text = "[x]<b>Shatter</b>\nDeal $4 damage.\n Deal $2 damage to\n all enemies.",
    set = "CATACLYSM",
    type = "spell",
    class = "mage",
    spell_school = "arcane",
    rarity = "rare",
    cost = 4,
    keywords = { "shatter" },
    target_mode = "required",
}

function card.targets(ctx, self)
    return ctx:enemy_characters(self)
end

function card.on_play(ctx, self, target)
    ctx:damage(target, 4)
    ctx:damage_all(ctx:enemy_characters(self), 2)
end

function card.on_shatter(ctx, self)
    local player = ctx:controller(self)
    ctx:move(self, "removed")
    ctx:give_card_at(player, "CATA_489t", 0)
    ctx:give_card_at(player, "CATA_489t2", 99)
end

card.tokens = {
    {
        id = "CATA_489t",
        name = "Arcane Flow",
        text = "<b>Shattered</b>\nDeal $4 damage.",
        set = "CATACLYSM",
        type = "spell",
        class = "mage",
        spell_school = "arcane",
        collectible = false,
        cost = 4,
        tags = { "shatter_fragment" },
        target_mode = "required",
        targets = function(ctx, self)
            return ctx:enemy_characters(self)
        end,
        on_play = function(ctx, self, target)
            ctx:damage(target, 4)
        end,
        triggers = {
            {
                event = "card_played",
                timing = "after",
                active_zones = { "hand" },
                condition = function(ctx, self, event)
                    if event.player ~= ctx:controller(self) then return false end
                    local hand = ctx:hand(event.player)
                    for index, entity in ipairs(hand) do
                        if entity == self then
                            return (index > 1 and ctx:entity(hand[index - 1]).card_id == "CATA_489t2")
                                or (index < #hand and ctx:entity(hand[index + 1]).card_id == "CATA_489t2")
                        end
                    end
                    return false
                end,
                effect = function(ctx, self, event)
                    local hand = ctx:hand(event.player)
                    local left_position = 0
                    local right = nil
                    for index, entity in ipairs(hand) do
                        if entity == self then left_position = index - 1 end
                        if ctx:entity(entity).card_id == "CATA_489t2" then right = entity end
                    end
                    if right ~= nil then
                        ctx:move(self, "removed")
                        ctx:move(right, "removed")
                        ctx:give_card_at(event.player, "CATA_489", left_position)
                    end
                end,
            },
        },
    },
    {
        id = "CATA_489t2",
        name = "Arcane Flow",
        text = "<b>Shattered</b>\nDeal $2 damage to\nall enemies.",
        set = "CATACLYSM",
        type = "spell",
        class = "mage",
        spell_school = "arcane",
        collectible = false,
        cost = 4,
        tags = { "shatter_fragment" },
        on_play = function(ctx, self)
            ctx:damage_all(ctx:enemy_characters(self), 2)
        end,
    },
}

return card
