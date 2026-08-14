local function death_pool(ctx, self)
    return ctx:minions_died(ctx:controller(self))
end

local card = {
    api_version = 1,
    id = "BRM_017",
    name = "Resurrect",
    text = "Summon a random friendly minion that died this game.",
    set = "BRM",
    type = "spell",
    class = "priest",
    rarity = "rare",
    spell_school = "holy",
    cost = 2,
    rules = {
        can_play = function(ctx, self, current)
            return current
                and #ctx:board(ctx:controller(self)) < 7
                and #death_pool(ctx, self) > 0
        end,
    },
}

function card.on_play(ctx, self)
    local pool = death_pool(ctx, self)
    if #pool > 0 then ctx:random_value(pool, "summon_resurrected_minion") end
end

function card.summon_resurrected_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
