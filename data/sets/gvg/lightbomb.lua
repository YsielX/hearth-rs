local card = {
    api_version = 1,
    id = "GVG_008",
    name = "Lightbomb",
    text = "Deal damage to each minion equal to its Attack.",
    set = "GVG",
    type = "spell",
    class = "priest",
    rarity = "epic",
    cost = 6,
    spell_school = "holy",
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.source == self
            end,
            effect = function(ctx, self, event)
                local spell_damage = 0
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    spell_damage = spell_damage + ctx:entity(minion).spell_damage
                end
                cardlib.effects.set_event_amount(ctx, event, ctx:entity(event.target).attack + spell_damage)
            end,
        },
    },
}

function card.on_play(ctx, self)
    local minions = ctx:minions()
    if #minions > 0 then
        -- A group keeps the damage simultaneous. Its provisional amount is replaced
        -- per target by the before trigger above, including this spell's damage bonus.
        cardlib.effects.damage_all(ctx, minions, 1)
    end
end

return card
