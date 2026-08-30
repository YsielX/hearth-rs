local card = {
    api_version = 1, id = "CFM_696", spell_school = "nature", name = "Devolve",
    text = "Transform all enemy minions into random ones that cost (1) less.",
    set = "GANGS", type = "spell", class = "shaman", rarity = "rare", cost = 2,
}
local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == "dormant" then return true end
    end
    return false
end
function card.on_play(ctx, self)
    for _, entity in ipairs(ctx:enemy_minions(self)) do
        if not dormant(ctx, entity) then
            ctx:set_data(self, "devolve_pending_" .. entity, 1)
            ctx:set_data(self, "devolve_cost_" .. entity, ctx:entity(entity).cost)
        end
    end
    ctx:continue_with("devolve_next")
end
function card.devolve_next(ctx, self)
    local target = nil
    for _, entity in ipairs(ctx:board(ctx:opponent(ctx:controller(self)))) do
        if ctx:get_data(self, "devolve_pending_" .. entity) == 1 then target = entity break end
    end
    if not target then return end
    ctx:set_data(self, "devolve_pending_" .. target, 0)
    ctx:set_data(self, "devolve_target", target)
    local wanted = math.max(0, ctx:get_data(self, "devolve_cost_" .. target) - 1)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == wanted then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "transform_target")
    else ctx:continue_with("devolve_next") end
end
function card.transform_target(ctx, self, card_id)
    local target = ctx:get_data(self, "devolve_target")
    if ctx:entity(target).zone == "board" then cardlib.effects.transform(ctx, target, card_id) end
    ctx:continue_with("devolve_next")
end
return card
