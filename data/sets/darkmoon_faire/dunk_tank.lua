local function targets(ctx, self) return ctx:characters() end

local function enemy_minions(ctx, self)
    local result = {}
    for _, entity in ipairs(ctx:enemy_characters(self)) do
        if ctx:entity(entity).type == "minion" then
            result[#result + 1] = entity
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "DMF_701",
    name = "Dunk Tank",
    text = "Deal $4 damage.\n<b>Corrupt:</b> Then deal $2 damage to all enemy minions.",
    set = "DARKMOON_FAIRE",
    type = "spell",
    class = "shaman",
    cost = 4,
    keywords = { "corrupt" },
    target_mode = "required",
    targets = targets,
}

function card.on_play(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end
function card.on_corrupt(ctx, self) cardlib.effects.transform(ctx, self, "DMF_701t") end

card.tokens = {
    {
        id = "DMF_701t",
        name = "Dunk Tank",
        text = "<b>Corrupted</b>\nDeal $4 damage, then\ndeal $2 damage to all enemy minions.",
        set = "DARKMOON_FAIRE",
        type = "spell",
        class = "shaman",
        cost = 4,
        target_mode = "required",
        targets = targets,
        on_play = function(ctx, self, target)
            cardlib.effects.damage(ctx, target, 4)
            cardlib.effects.damage_all(ctx, enemy_minions(ctx, self), 2)
        end,
    },
}

return card
