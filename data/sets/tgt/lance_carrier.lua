local card = {
    api_version = 1, id = "AT_084", name = "Lance Carrier",
    text = "<b>Battlecry:</b> Give a friendly minion +2 Attack.",
    set = "TGT", type = "minion", rarity = "common", cost = 2, attack = 1, health = 2,
    keywords = { "battlecry" }, target_mode = "required_if_available",
}

function card.targets(ctx, self) return ctx:board(ctx:controller(self)) end
function card.on_battlecry(ctx, self, target)
    if target then ctx:buff(target, 2, 0) end
end

return card
