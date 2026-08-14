local card = {
    api_version = 1, id = "GVG_090", name = "Madder Bomber",
    text = "<b>Battlecry:</b> Deal 6 damage randomly split between all other characters.",
    set = "GVG", type = "minion", rarity = "rare", cost = 5, attack = 5, health = 4,
    keywords = { "battlecry" },
}
local function throw(ctx, self)
    local targets = {}
    for _, target in ipairs(ctx:characters()) do
        if target ~= self then targets[#targets + 1] = target end
    end
    if #targets > 0 then ctx:random_entity(targets, "hit") end
end
function card.on_battlecry(ctx, self)
    ctx:set_data(self, "bombs_left", 6)
    throw(ctx, self)
end
function card.hit(ctx, self, target)
    ctx:damage(target, 1)
    local left = ctx:get_data(self, "bombs_left") - 1
    ctx:set_data(self, "bombs_left", left)
    if left > 0 then ctx:continue_with("throw_next") end
end
function card.throw_next(ctx, self) throw(ctx, self) end
return card
