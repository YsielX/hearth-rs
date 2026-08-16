local card = {
    api_version = 1,
    id = "GVG_015",
    name = "Darkbomb",
    text = "Deal $3 damage to a character. If it dies, draw a Shadow spell.",
    set = "GVG",
    type = "spell",
    class = "warlock",
    rarity = "common",
    spell_school = "shadow",
    cost = 2,
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 3)
    ctx:continue_with_entity("draw_shadow_spell_if_dead", target)
end

function card.draw_shadow_spell_if_dead(ctx, self, target)
    if ctx:entity(target).zone ~= "graveyard" then return end
    local player = ctx:controller(self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if definition.type == "spell" and definition.spell_school == "shadow" then
            table.insert(candidates, entity)
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "draw_selected_shadow_spell") end
end

function card.draw_selected_shadow_spell(ctx, self, selected)
    ctx:move(selected, "deck_top")
    ctx:draw(ctx:controller(self), 1)
end

return card
