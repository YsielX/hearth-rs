local card = {
    api_version = 1,
    id = "FP1_020", spell_school = "holy",
    name = "Avenge",
    text = "<b>Secret:</b> When one of your minions dies, give a random friendly minion +3/+2.",
    set = "NAXX",
    type = "spell",
    class = "paladin",
    rarity = "common",
    cost = 1,
    keywords = { "secret" },
}

card.triggers = {
    {
        event = "entity_died",
        timing = "after",
        active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and ctx:entity(event.entity).type == "minion"
                and #ctx:friendly_minions(self) > 0
        end,
        effect = function(ctx, self, event)
            ctx:reveal_secret(self)
            ctx:continue_with("begin_avenge")
        end,
    },
}

function card.begin_avenge(ctx, self)
    -- Trigger collection for one death checkpoint observes a stable snapshot.
    -- This marker makes simultaneous deaths consume the Secret only once.
    if ctx:get_data(self, "triggered") == 1 then return end
    ctx:set_data(self, "triggered", 1)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "avenge_minion") end
end

function card.avenge_minion(ctx, self, target)
    ctx:buff(target, 3, 2)
end

return card
