local card = {
    api_version = 1,
    id = "TTN_735", spell_school = "frost",
    name = "Northern Navigation",
    text = "<b>Discover</b> a spell from your deck. If it's\na Frost spell, <b>Freeze</b> a random enemy minion.",
    set = "TITANS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    cost = 2,
    rune_cost = { frost = 1 },
    keywords = { "discover" },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local spells = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
    end
    if #spells > 0 then
        ctx:discover_entities(
            player,
            ctx:localize(
                "Discover a spell from your deck",
                "从你的牌库中发现一张法术牌",
                "從你的牌堆中發現一張法術牌"
            ),
            spells,
            3,
            "draw_navigated_spell"
        )
    end
end

function card.draw_navigated_spell(ctx, self, spell)
    local definition = ctx:card_definition(ctx:entity(spell).card_id)
    ctx:draw_entity(ctx:controller(self), spell)
    if definition.spell_school == "frost" then
        local enemies = ctx:enemy_minions(self)
        if #enemies > 0 then ctx:random_entity(enemies, "freeze_northern_target") end
    end
end

function card.freeze_northern_target(ctx, self, target)
    ctx:freeze(target)
end

return card
